export interface AppError {
  code: string;
  message: string;
  details: string;
  severity: ErrorSeverity;
  timestamp: string;
}

export type ErrorSeverity = "info" | "warning" | "error" | "critical";

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface SortOption {
  field: string;
  order: "asc" | "desc";
}

export interface FilterOption {
  field: string;
  operator: "eq" | "ne" | "gt" | "lt" | "contains" | "starts_with";
  value: unknown;
}

export interface SizeInfo {
  bytes: number;
  formatted: string;
}

export interface TimeInterval {
  startTime: string;
  endTime: string;
  durationMs: number;
}
