import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";

const flameIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.24 17 7.07A8 8 0 0117.657 18.657z"/></svg>`;
const snowIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 2v20M2 12h20M4.93 4.93l14.14 14.14M19.07 4.93L4.93 19.07"/></svg>`;
const dropletIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 2l5.66 8.66a7 7 0 11-11.31 0L12 2z"/></svg>`;
const fanIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3" stroke-width="2"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5V2M12 22v-3M5 12H2M22 12h-3M6.34 6.34L4.22 4.22M19.78 19.78l-2.12-2.12M6.34 17.66l-2.12 2.12M19.78 4.22l-2.12 2.12"/></svg>`;
const sunIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><circle cx="12" cy="12" r="5" stroke-width="2"/><line x1="12" y1="1" x2="12" y2="3" stroke-width="2"/><line x1="12" y1="21" x2="12" y2="23" stroke-width="2"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64" stroke-width="2"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78" stroke-width="2"/><line x1="1" y1="12" x2="3" y2="12" stroke-width="2"/><line x1="21" y1="12" x2="23" y2="12" stroke-width="2"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36" stroke-width="2"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22" stroke-width="2"/></svg>`;
const bulbIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 18h6M10 22h4M12 2a7 7 0 00-4 12.74V17h8v-2.26A7 7 0 0012 2z"/></svg>`;

export default function SystemsTab() {
  const navigate = useNavigate();
  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label="Comfort">
          <RibbonButton icon={flameIcon} label="Verwarming" onClick={() => navigate("/systems")} />
          <RibbonButton icon={snowIcon} label="Koeling" onClick={() => navigate("/systems")} />
          <RibbonButton icon={dropletIcon} label="Tapwater" onClick={() => navigate("/systems")} />
        </RibbonGroup>
        <RibbonGroup label="Lucht">
          <RibbonButton icon={fanIcon} label="Ventilatie" onClick={() => navigate("/systems")} />
        </RibbonGroup>
        <RibbonGroup label="Energie">
          <RibbonButton icon={sunIcon} label="PV" onClick={() => navigate("/systems")} />
          <RibbonButton icon={bulbIcon} label="Verlichting" onClick={() => navigate("/systems")} />
        </RibbonGroup>
      </div>
    </div>
  );
}
