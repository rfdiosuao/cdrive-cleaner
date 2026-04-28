import { ref } from "vue";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: number;
  type: ToastType;
  message: string;
  duration: number;
}

const toasts = ref<Toast[]>([]);
let nextId = 0;

export function useToast() {
  function show(type: ToastType, message: string, duration: number = 3000) {
    const id = nextId++;
    toasts.value.push({ id, type, message, duration });
    if (duration > 0) {
      setTimeout(() => {
        remove(id);
      }, duration);
    }
  }

  function remove(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  function success(message: string, duration?: number) {
    show("success", message, duration);
  }

  function error(message: string, duration?: number) {
    show("error", message, duration);
  }

  function warning(message: string, duration?: number) {
    show("warning", message, duration);
  }

  function info(message: string, duration?: number) {
    show("info", message, duration);
  }

  return {
    toasts,
    show,
    remove,
    success,
    error,
    warning,
    info,
  };
}
