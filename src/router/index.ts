import { createRouter, createWebHistory } from "vue-router";
import Layout from "../components/Layout.vue";

const routes = [
  {
    path: "/",
    component: Layout,
    children: [
      {
        path: "",
        name: "Dashboard",
        component: () => import("../views/Dashboard.vue"),
      },
      {
        path: "/scan",
        name: "ScanResult",
        component: () => import("../views/ScanResult.vue"),
      },
      {
        path: "/clean",
        name: "CleanExecute",
        component: () => import("../views/CleanExecute.vue"),
      },
      {
        path: "/large-file",
        name: "LargeFile",
        component: () => import("../views/LargeFile.vue"),
      },
      {
        path: "/software-move",
        name: "SoftwareMove",
        component: () => import("../views/SoftwareMove.vue"),
      },
      {
        path: "/ai-migrate",
        name: "AiMigrate",
        component: () => import("../views/AiMigrate.vue"),
      },
      {
        path: "/settings",
        name: "Settings",
        component: () => import("../views/Settings.vue"),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
