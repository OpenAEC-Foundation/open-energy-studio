import { useNavigate } from "react-router-dom";

import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";

import { isTauri } from "../../lib/backend";
import { useProjectStore } from "../../store/projectStore";
import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { plusIcon, buildingIcon } from "./icons";

const folderOpenIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h7a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2l-2 7"/></svg>`;
const saveIcon = `<svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"/></svg>`;

const FILTERS = [
  { name: "Open Energy Studio project", extensions: ["oes.json", "oes", "json"] },
];

export default function ProjectTab() {
  const navigate = useNavigate();
  const project = useProjectStore((s) => s.project);
  const filePath = useProjectStore((s) => s.filePath);
  const newProject = useProjectStore((s) => s.newProject);
  const loadProject = useProjectStore((s) => s.loadProject);
  const saveProject = useProjectStore((s) => s.saveProject);

  const handleNew = async () => {
    await newProject("Nieuw project");
    navigate("/project");
  };

  const handleOpen = async () => {
    if (!isTauri()) return;
    const picked = await openDialog({ multiple: false, directory: false, filters: FILTERS });
    if (typeof picked === "string") {
      await loadProject(picked);
      navigate("/project");
    }
  };

  const handleSave = async () => {
    if (filePath) {
      await saveProject();
      return;
    }
    if (!isTauri()) return;
    const picked = await saveDialog({ defaultPath: "project.oes.json", filters: FILTERS });
    if (typeof picked === "string") await saveProject(picked);
  };

  return (
    <div className="ribbon-content">
      <div className="ribbon-groups">
        <RibbonGroup label="Bestand">
          <RibbonButton icon={plusIcon} label="Nieuw" onClick={handleNew} />
          <RibbonButton icon={folderOpenIcon} label="Openen" onClick={handleOpen} />
          <RibbonButton
            icon={saveIcon}
            label="Opslaan"
            disabled={!project}
            onClick={handleSave}
          />
        </RibbonGroup>
        <RibbonGroup label="Project">
          <RibbonButton
            icon={buildingIcon}
            label="Projectgegevens"
            onClick={() => navigate("/project")}
          />
        </RibbonGroup>
      </div>
    </div>
  );
}
