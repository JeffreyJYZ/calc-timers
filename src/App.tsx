import { useEffect, useState } from "react";
import { TabBar, type Tab } from "./components/TabBar";
import { Calculator } from "./components/Calculator";
import { TimerList } from "./components/TimerList";
import { useTimerStore } from "./store/timerStore";
import { requestNotificationPermission } from "./lib/notifications";

const STORAGE_KEY = "calctimers.activeTab";

function readTab(): Tab {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === "timers" ? "timers" : "calc";
  } catch {
    return "calc";
  }
}

function App() {
  const [tab, setTab] = useState<Tab>(() => readTab());
  const timerCount = useTimerStore((s) => s.timers.length);

  useEffect(() => {
    requestNotificationPermission();
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, tab);
    } catch {
      /* ignore */
    }
  }, [tab]);

  return (
    <div className="flex h-full w-full flex-col">
      <TabBar active={tab} onChange={setTab} timerCount={timerCount} />
      <main className="flex-1 overflow-hidden">
        {tab === "calc" ? <Calculator /> : <TimerList />}
      </main>
    </div>
  );
}

export default App;
