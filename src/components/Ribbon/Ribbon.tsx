import { useState, useRef, useEffect, useCallback } from "react";
import { useLocation, useNavigate } from "react-router-dom";

import RibbonTab from "./RibbonTab";
import ProjectTab from "./ProjectTab";
import ZonesTab from "./ZonesTab";
import ConstructiesTab from "./ConstructiesTab";
import SystemsTab from "./SystemsTab";
import ResultatenTab from "./ResultatenTab";
import VerificatieTab from "./VerificatieTab";
import "./Ribbon.css";

interface RibbonProps {
  onFileTabClick?: () => void;
}

const TABS = [
  "project",
  "zones",
  "constructies",
  "systems",
  "resultaten",
  "verificatie",
] as const;
type TabId = (typeof TABS)[number];

const TAB_LABELS: Record<TabId, string> = {
  project: "Project",
  zones: "Rekenzones",
  constructies: "Constructies",
  systems: "Installaties",
  resultaten: "Resultaten",
  verificatie: "Verificatie",
};

const ROUTE_TO_TAB: Record<string, TabId> = {
  "/project": "project",
  "/zones": "zones",
  "/constructies": "constructies",
  "/systems": "systems",
  "/results": "resultaten",
  "/verify": "verificatie",
};

const TAB_TO_ROUTE: Record<TabId, string> = {
  project: "/project",
  zones: "/zones",
  constructies: "/constructies",
  systems: "/systems",
  resultaten: "/results",
  verificatie: "/verify",
};

export default function Ribbon({ onFileTabClick }: RibbonProps) {
  const location = useLocation();
  const navigate = useNavigate();

  const tabFromRoute = ROUTE_TO_TAB[location.pathname] ?? "project";
  const [activeTab, setActiveTab] = useState<TabId>(tabFromRoute);
  const [prevTab, setPrevTab] = useState<TabId | null>(null);
  const [animating, setAnimating] = useState(false);
  const [direction, setDirection] = useState<"left" | "right">("right");
  const tabsRef = useRef<HTMLDivElement>(null);
  const borderRef = useRef<HTMLDivElement>(null);
  const gapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const newTab = ROUTE_TO_TAB[location.pathname];
    if (newTab && newTab !== activeTab) setActiveTab(newTab);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [location.pathname]);

  const updateHighlight = useCallback(() => {
    const tabsEl = tabsRef.current;
    const borderEl = borderRef.current;
    const gapEl = gapRef.current;
    if (!tabsEl || !borderEl || !gapEl) return;
    const activeEl = tabsEl.querySelector(".ribbon-tab.active") as HTMLElement | null;
    if (!activeEl) {
      borderEl.style.opacity = "0";
      gapEl.style.opacity = "0";
      return;
    }
    const tabsRect = tabsEl.getBoundingClientRect();
    const activeRect = activeEl.getBoundingClientRect();
    const left = activeRect.left - tabsRect.left;
    const top = activeRect.top - tabsRect.top;
    const width = activeRect.width;
    const height = activeRect.height;
    borderEl.style.opacity = "1";
    borderEl.style.left = `${left}px`;
    borderEl.style.top = `${top}px`;
    borderEl.style.width = `${width}px`;
    borderEl.style.height = `${height}px`;
    gapEl.style.opacity = "1";
    gapEl.style.left = `${left + 1}px`;
    gapEl.style.width = `${width - 2}px`;
  }, []);

  const switchTab = useCallback(
    (newTab: TabId) => {
      if (newTab === activeTab) return;
      const oldIndex = TABS.indexOf(activeTab);
      const newIndex = TABS.indexOf(newTab);
      setDirection(newIndex > oldIndex ? "right" : "left");
      setPrevTab(activeTab);
      setActiveTab(newTab);
      setAnimating(true);
      navigate(TAB_TO_ROUTE[newTab]);
    },
    [activeTab, navigate],
  );

  useEffect(() => {
    updateHighlight();
    requestAnimationFrame(updateHighlight);
  }, [activeTab, updateHighlight]);

  useEffect(() => {
    window.addEventListener("resize", updateHighlight);
    return () => window.removeEventListener("resize", updateHighlight);
  }, [updateHighlight]);

  useEffect(() => {
    if (!animating) return;
    const t = setTimeout(() => {
      setAnimating(false);
      setPrevTab(null);
    }, 250);
    return () => clearTimeout(t);
  }, [animating]);

  const renderContent = (tab: TabId) => {
    switch (tab) {
      case "project":
        return <ProjectTab />;
      case "zones":
        return <ZonesTab />;
      case "constructies":
        return <ConstructiesTab />;
      case "systems":
        return <SystemsTab />;
      case "resultaten":
        return <ResultatenTab />;
      case "verificatie":
        return <VerificatieTab />;
    }
  };

  return (
    <div className="ribbon-container">
      <div className="ribbon-tabs" ref={tabsRef}>
        <RibbonTab label="Bestand" isFileTab onClick={() => onFileTabClick?.()} />
        {TABS.map((tab) => (
          <RibbonTab
            key={tab}
            label={TAB_LABELS[tab]}
            isActive={activeTab === tab}
            onClick={() => switchTab(tab)}
          />
        ))}
        <div className="ribbon-tab-border" ref={borderRef} />
        <div className="ribbon-tab-gap" ref={gapRef} />
      </div>

      <div className="ribbon-content-wrapper">
        {animating && prevTab && (
          <div
            className={`ribbon-content-panel ribbon-panel-exit-${direction}`}
            key={`prev-${prevTab}`}
          >
            {renderContent(prevTab)}
          </div>
        )}
        <div
          className={`ribbon-content-panel${animating ? ` ribbon-panel-enter-${direction}` : ""}`}
          key={`active-${activeTab}`}
        >
          {renderContent(activeTab)}
        </div>
      </div>
    </div>
  );
}
