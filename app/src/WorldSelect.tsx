import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import { savesDir } from "./openWorld";

export interface WorldSelectProps {
  worlds?: WorldInfo[];
  selected?: string;
  onChange: (info?: WorldInfo) => void;
}

export interface WorldInfo {
  level: {
    Data: {
      LevelName: string;
      Version: {
        Name: string;
      };
      SpawnX: number;
      SpawnZ: number;
    };
  };
  dir: string;
}

export function useWorldSelect(): WorldInfo[] | undefined {
  const [worlds, setWorlds] = useState<WorldInfo[] | undefined>();

  useEffect(() => {
    const inner = async () => {
      try {
        const saves = await savesDir();
        if (!saves) {
          throw new Error(
            "Could not determine saves directory on this platform"
          );
        }
        const worlds = (await invoke("world_list", {
          dir: saves,
        })) as WorldInfo[];

        setWorlds(worlds);
      } catch (err) {
        console.error(err);
      }
    };
    inner();
  }, []);

  return worlds;
}

export function WorldSelect({ worlds, selected, onChange }: WorldSelectProps) {
  const handleChange = (ev: React.ChangeEvent<HTMLSelectElement>) => {
    let i = Number(ev.target.value);
    onChange(worlds && worlds[i]);
  };

  const selectedValue =
    worlds?.findIndex((info) => {
      return info.dir === selected;
    }) ?? -1;

  return (
    <label>
      <span>Save:</span>
      <select onChange={handleChange} value={selectedValue}>
        <option key={-1} value={-1}>
          None
        </option>
        {worlds?.map((info, i) => (
          <option key={i} value={i}>
            {`${info.level.Data.LevelName} (${info.level.Data.Version.Name})`}
          </option>
        ))}
      </select>
    </label>
  );
}
