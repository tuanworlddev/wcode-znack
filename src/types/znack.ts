// Mirrors the serde structs in src-tauri/src/znack/models.rs (camelCase).

export interface ZnackSettings {
  trueApiBaseUrl: string;
  suzBaseUrl: string;
  omsId: string;
  omsConnection: string;
  participantInn: string;
  producerInn: string;
  ownerInn: string;
  cryptcpPath: string;
  certThumbprint: string;
  certLabel: string;
  cryptoproTimeoutSeconds: number;
  documentType: string;
  documentNumber: string;
  documentDate: string;
  autoIntroduction: boolean;
}

export interface ZnackPipeline {
  id: number;
  gtin: string;
  quantity: number;
  orderId: number | null;
  stage: string;
  errorMessage: string | null;
  updatedAt: string;
}

export interface ZnackProduct {
  gtin: string;
  productName: string;
  tnVed: string;
  goodMarkFlag: boolean | null;
  goodTurnFlag: boolean | null;
  cardStatus: string;
  cardDetailedStatus: string;
  syncedAt: string;
  available: number;
  reserved: number;
  consumed: number;
  pipeline: ZnackPipeline | null;
}

export interface ZnackMappingRule {
  id: number;
  gtin: string;
  subjectName: string;
  genderValue: string;
  wildcardGender: boolean;
}

export interface ZnackCertificate {
  thumbprint: string;
  subject: string;
  inn: string;
  notAfter: string;
}

export interface CategoryGender {
  subjectName: string;
  gender: string;
  productCount: number;
}

/** Resolve a GTIN for (category, gender) from the rule list — mirrors the
 *  backend resolution in znack/db.rs. */
export function resolveGtin(
  rules: ZnackMappingRule[],
  subjectName: string,
  gender: string
): string | null {
  const forSubject = rules.filter((r) => r.subjectName === subjectName);
  if (forSubject.length === 0) return null;
  const g = gender.trim().toLowerCase();
  // Exact match first — including the "" gender for products without Пол.
  const exact = forSubject.find(
    (r) => !r.wildcardGender && r.genderValue.trim().toLowerCase() === g
  );
  if (exact) return exact.gtin;
  const wildcard = forSubject.find((r) => r.wildcardGender);
  if (wildcard) return wildcard.gtin;
  if (forSubject.length === 1 && !g) return forSubject[0].gtin;
  return null;
}

export const ACTIVE_STAGES = new Set([
  "VALIDATING",
  "CREATING_ORDER",
  "POLLING_ORDER",
  "DOWNLOADING_CODES",
  "WAITING_INTRODUCTION_READINESS",
  "SUBMITTING_INTRODUCTION",
  "POLLING_INTRODUCTION",
]);

export const STAGE_LABELS: Record<string, string> = {
  VALIDATING: "Đang kiểm tra",
  CREATING_ORDER: "Đang tạo đơn (cần xử lý tay nếu kẹt)",
  POLLING_ORDER: "Chờ СУЗ phát mã",
  DOWNLOADING_CODES: "Đang tải mã",
  WAITING_INTRODUCTION_READINESS: "Chờ sẵn sàng lưu thông",
  SUBMITTING_INTRODUCTION: "Đang gửi lưu thông",
  POLLING_INTRODUCTION: "Chờ xác nhận lưu thông",
  INTRODUCED: "Đã lưu thông",
  COMPLETED: "Hoàn tất",
  FAILED: "Thất bại",
};
