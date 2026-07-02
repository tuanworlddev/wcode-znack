// Shared "print a supply" flow: fetch print data (which reserves KIZ codes),
// apply the saved sort order, build + open BOTH files (labels + picking list),
// then commit the KIZ reservation — or release it if anything failed.

import { api, errMsg } from "../api/tauri";
import { buildPickingPdfBase64, buildSupplyPdfBase64 } from "../pdf";
import { loadSortPrefs, sortPrintOrders } from "./supplySort";
import type { Notify } from "../App";

export async function printSupply(
  supplyId: string,
  supplyName: string,
  notify: Notify
): Promise<void> {
  let kizToken: string | null = null;
  try {
    notify(`Đang chuẩn bị nhãn cho "${supplyName}"...`, "info");
    const data = await api.getSupplyPrintData(supplyId);
    kizToken = data.kizToken;
    const orders = data.orders;
    if (orders.length === 0) {
      notify("Lô hàng chưa có đơn nào.", "err");
      return;
    }
    const sorted = sortPrintOrders(orders, loadSortPrefs());
    const { base64, missingStickers } = await buildSupplyPdfBase64(sorted);
    const picking = await buildPickingPdfBase64(supplyId, sorted);
    await api.saveAndOpenPdf(supplyId, base64);
    await api.saveAndOpenPdf(supplyId, picking, "picking");

    // The PDFs are saved — the reserved KIZ codes are now used up.
    if (kizToken) {
      try {
        await api.finishKizReservation(kizToken, true);
      } catch (e) {
        // Codes stay RESERVED until app restart; tell the user instead of
        // silently losing track of them.
        notify(`Không chốt được kho mã KIZ: ${errMsg(e)}`, "err");
      }
      kizToken = null;
    }

    // Push each printed KIZ to its WB assembly order (meta/sgtin).
    const assignments = sorted
      .filter((o) => o.kizCode)
      .map((o) => ({ orderId: o.orderId, sgtin: o.kizCode }));
    let pushNote = "";
    if (assignments.length > 0) {
      try {
        const result = await api.assignOrderSgtins(assignments);
        pushNote =
          result.failed.length > 0
            ? `Đã gắn SGTIN lên WB cho ${result.ok}/${assignments.length} đơn. Lỗi: ${result.failed.join("; ")}`
            : `Đã gắn SGTIN lên WB cho ${result.ok} đơn`;
      } catch (e) {
        pushNote = `Gắn SGTIN lên WB thất bại: ${errMsg(e)}`;
      }
    }

    const notes: string[] = [];
    if (missingStickers > 0) notes.push(`${missingStickers} đơn chưa có nhãn vận đơn`);
    if (pushNote) notes.push(pushNote);
    notify(
      notes.length > 0
        ? `Đã mở file in + file nhặt hàng (${sorted.length} đơn). ${notes.join(". ")}.`
        : `Đã tạo file in + file nhặt hàng cho ${sorted.length} đơn.`,
      pushNote.includes("Lỗi") || pushNote.includes("thất bại") ? "err" : notes.length > 0 ? "info" : "ok"
    );
  } catch (e) {
    // Return unused reserved codes to the pool.
    if (kizToken) {
      await api.finishKizReservation(kizToken, false).catch(() => {});
    }
    notify(`In nhãn thất bại: ${errMsg(e)}`, "err");
  }
}
