import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { plusIcon, layersIcon } from "./icons";

const windowIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2" stroke-width="2"/><line x1="3" y1="12" x2="21" y2="12" stroke-width="2"/><line x1="12" y1="3" x2="12" y2="21" stroke-width="2"/></svg>`;
const bridgeIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 17l6-6 4 4 8-8M14 7h7v7"/></svg>`;

export default function ConstructiesTab() {
  const navigate = useNavigate();
  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label="Envelope">
          <RibbonButton
            icon={layersIcon}
            label="Constructies"
            onClick={() => navigate("/constructies")}
          />
          <RibbonButton icon={plusIcon} label="Toevoegen" onClick={() => navigate("/constructies")} />
        </RibbonGroup>
        <RibbonGroup label="Bijzonder">
          <RibbonButton
            icon={windowIcon}
            label="Ramen / Deuren"
            onClick={() => navigate("/constructies")}
          />
          <RibbonButton
            icon={bridgeIcon}
            label="Koudebruggen"
            onClick={() => navigate("/constructies")}
          />
        </RibbonGroup>
      </div>
    </div>
  );
}
