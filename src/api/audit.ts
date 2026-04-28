import { invokeCommand } from "./index";

export interface AuditLogEntry {
  id: string;
  action: string;
  targetPath: string;
  detail: string;
  riskLevel: string;
  timestamp: string;
}

export async function auditLogList(
  actionFilter?: string,
  limit = 100,
  offset = 0,
): Promise<AuditLogEntry[]> {
  return invokeCommand<AuditLogEntry[]>("audit_log_list", {
    actionFilter,
    limit,
    offset,
  });
}
