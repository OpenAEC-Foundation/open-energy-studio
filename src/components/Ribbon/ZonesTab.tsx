import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { plusIcon } from "./icons";

const homeIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l9-9 9 9M5 10v10a1 1 0 001 1h3m10-11v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/></svg>`;

export default function ZonesTab() {
  const navigate = useNavigate();
  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label="Rekenzones">
          <RibbonButton icon={plusIcon} label="Toevoegen" onClick={() => navigate("/zones")} />
          <RibbonButton icon={homeIcon} label="Heel gebouw" onClick={() => navigate("/zones")} />
        </RibbonGroup>
      </div>
    </div>
  );
}
