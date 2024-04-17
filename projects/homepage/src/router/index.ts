import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
    {
        path: "/",
        name: "home",
        component: () => import("@/views/HomeView.vue"),
    },
    {
        path: "/i",
        name: "deployment",
        component: () => import("@/views/DeploymentView.vue"),
    },
    {
        path: "/t",
        name: "terminology",
        component: () => import("@/views/TerminologyView.vue"),
    },
    {
        path: "/d",
        name: "docs",
        component: () => import("@/views/DocsView.vue"),
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
