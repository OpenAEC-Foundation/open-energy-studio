import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";

const checkIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>`;
const playIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><polygon stroke-linecap="round" stroke-linejoin="round" stroke-width="2" points="5 3 19 12 5 21 5 3"/></svg>`;

export default function VerificatieTab() {
  const navigate = useNavigate();
  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label="Referentie">
          <RibbonButton
            icon={checkIcon}
            label="Cases"
            onClick={() => navigate("/verify")}
          />
          <RibbonButton icon={playIcon} label="Alles draaien" disabled />
        </RibbonGroup>
      </div>
    </div>
  );
}
