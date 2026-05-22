import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";

import { AppShell } from "./components/layout/AppShell";
import ProjectSetup from "./pages/ProjectSetup";
import Zones from "./pages/Zones";
import Constructies from "./pages/Constructies";
import Systems from "./pages/Systems";
import Results from "./pages/Results";
import Verificatie from "./pages/Verificatie";

/**
 * Application root — Tauri shell with router-driven NTA 8800 pages.
 *
 * Modeled after `open-heatloss-studio`'s App.tsx with the OpenAEC style book
 * tokens (themes.css + Tailwind). Tab content and page contents are
 * NTA 8800-oriented; the visual chrome (TitleBar, Ribbon, Sidebar,
 * StatusBar) is the heatloss-studio pattern.
 */
export default function App() {
  return (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<Navigate to="/project" replace />} />
          <Route path="/project" element={<ProjectSetup />} />
          <Route path="/zones" element={<Zones />} />
          <Route path="/constructies" element={<Constructies />} />
          <Route path="/systems" element={<Systems />} />
          <Route path="/results" element={<Results />} />
          <Route path="/verify" element={<Verificatie />} />
        </Routes>
      </AppShell>
    </BrowserRouter>
  );
}
