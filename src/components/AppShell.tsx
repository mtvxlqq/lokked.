import { Outlet } from "react-router";

import { Logo } from "@/components/nav/Logo";
import { Sidebar } from "@/components/nav/Sidebar";
import { TabBar } from "@/components/nav/TabBar";

/**
 * Каркас интерфейса: навигация плюс область экрана.
 *
 * До 768px — колонка с нижним таб-баром, выше — строка с боковой панелью.
 * Логотип живёт либо в шапке узкого экрана, либо в сайдбаре: показывать его
 * дважды незачем.
 *
 * Контент не растягивается шире `max-w-app` — на широком мониторе строки
 * списков иначе превращаются в километровые полосы.
 */
export function AppShell() {
  return (
    <div className="flex min-h-screen flex-col md:flex-row">
      <Sidebar />

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="px-5 pt-6.5 sm:px-14 sm:pt-11 md:hidden">
          <Logo className="text-17" />
        </header>

        <main className="mx-auto flex w-full max-w-app flex-1 flex-col px-5 pt-4.5 pb-6.5 sm:px-14 sm:pb-11 md:pt-11">
          <Outlet />
        </main>

        <TabBar />
      </div>
    </div>
  );
}
