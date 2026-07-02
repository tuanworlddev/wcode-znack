export function cargoLabel(t: number): string {
  // Only show a chip for oversized/bulky cargo; normal goods (МГТ) get none.
  switch (t) {
    case 2:
      return "Hàng lớn";
    case 3:
      return "Hàng cồng kềnh";
    default:
      return "";
  }
}

export function deliveryLabel(pickup: boolean): string {
  return pickup ? "Giao ra điểm nhận" : "Giao về kho WB";
}

export function fmtDate(s: string | null | undefined): string {
  if (!s) return "";
  const d = new Date(s);
  return isNaN(d.getTime()) ? "" : d.toLocaleDateString("vi-VN");
}

export function fmtPrice(priceKop: number): string {
  const v = Math.round((priceKop || 0) / 100);
  return v.toLocaleString("ru-RU") + " ₽";
}

/** "2 giờ 46 phút trước" style relative time. */
export function timeAgo(s: string): string {
  const d = new Date(s).getTime();
  if (isNaN(d)) return "";
  let sec = Math.floor((Date.now() - d) / 1000);
  if (sec < 60) return "vừa xong";
  const days = Math.floor(sec / 86400);
  sec -= days * 86400;
  const h = Math.floor(sec / 3600);
  sec -= h * 3600;
  const m = Math.floor(sec / 60);
  if (days > 0) return `${days} ngày ${h} giờ trước`;
  if (h > 0) return `${h} giờ ${m} phút trước`;
  return `${m} phút trước`;
}
