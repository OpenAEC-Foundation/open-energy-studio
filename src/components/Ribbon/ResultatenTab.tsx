import { useNavigate } from "react-router-dom";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { useProjectStore } from "../../store/projectStore";
import { calculatorIcon, chartIcon, reportIcon, exportIcon } from "./icons";

export default function ResultatenTab() {
  const navigate = useNavigate();
  const project = useProjectStore((s) => s.project);
  const calculate = useProjectStore((s) => s.calculate);
  const isCalculating = useProjectStore((s) => s.isCalculating);

  const handleCalculate = async () => {
    await calculate();
    navigate("/results");
  };

  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label="Berekening">
          <RibbonButton
            icon={calculatorIcon}
            label="Bereken"
            disabled={!project || isCalculating}
            onClick={handleCalculate}
          />
          <RibbonButton
            icon={chartIcon}
            label="Resultaten"
            onClick={() => navigate("/results")}
          />
        </RibbonGroup>
        <RibbonGroup label="Rapport">
          <RibbonButton icon={reportIcon} label="PDF" disabled />
        </RibbonGroup>
        <RibbonGroup label="Export">
          <RibbonButton icon={exportIcon} label="Vabi" disabled />
          <RibbonButton icon={exportIcon} label="Uniec" disabled />
        </RibbonGroup>
      </div>
    </div>
  );
}
