import { useEffect, useRef, useState } from "react";
import "./App.css";
import { open } from "@tauri-apps/api/dialog";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

import { MapContainer } from "react-leaflet";
import { AnvilLayer } from "./AnvilLayer";
import { Ribbon } from "./Ribbon";
import { WorldInfo, WorldSelect, useWorldSelect } from "./WorldSelect";
import { invoke } from "@tauri-apps/api";

function useWorld(
  worldDir?: string,
  worlds?: WorldInfo[]
): WorldInfo | undefined {
  const [info, setInfo] = useState<WorldInfo | undefined>();
  // If worldDir is not in the list of saves, load the world info
  // manually.
  useEffect(() => {
    const info = worlds?.find((info) => info.dir === worldDir);
    if (info) {
      setInfo(info);
    } else {
      const inner = async () => {
        const resp = (await invoke("world_info", {
          dir: worldDir,
        })) as WorldInfo | undefined;
        if (resp) {
          setInfo(resp);
        }
      };
      inner();
    }
  }, [worldDir]);

  return info;
}

function App() {
  const [worldDir, setWorldDir] = useState<string | undefined>();
  const worlds = useWorldSelect();
  const worldInfo = useWorld(worldDir, worlds);

  const [trustHeights, setTrustHeights] = useState(true);
  const [dimension, setDimension] = useState("overworld");
  const mapRef = useRef<L.Map | null>(null);

  async function handleOpen() {
    const selected = await open({
      multiple: false,
      directory: true,
    });

    if (selected && !Array.isArray(selected)) {
      setWorldDir(selected);
    }
  }

  function handleWorldSelect(world?: WorldInfo) {
    setWorldDir(world && world.dir);
  }

  function handleTrustHeightChange(ev: React.ChangeEvent<HTMLInputElement>) {
    setTrustHeights((prev) => !prev);
  }

  function handleDimensionChange(ev: React.ChangeEvent<HTMLSelectElement>) {
    setDimension(ev.target.value);
    if (mapRef.current) {
      mapRef.current;
    }
  }

  // TODO: Some way for super zoom out? Currently causes way too many tile
  // requests.
  // TODO: Zoom in is blurry. Feels like it shouldn't be on MacOS's webview. But
  // it is.
  // TODO: Just list saves in drop down or something rather than file dialog
  // TODO: Show coordinates.
  // TODO: Distingush broken vs missing regions.
  // TODO: Fix region loading indication not disappearing properly.
  // TODO: Search for item! eg specific enchantment or name
  // TODO: Add players! Show inventory, enderchest, acheivements.
  // TODO: Fly to player. Could be any dimension.
  // TODO: Search for block/entity
  return (
    <div className="container">
      <Ribbon>
        <WorldSelect
          selected={worldDir}
          worlds={worlds}
          onChange={handleWorldSelect}
        />
        <label>
          <span>Dimension:</span>
          <select onChange={handleDimensionChange}>
            <option value="overworld">Overworld</option>
            <option value="nether">Nether</option>
            <option value="end">The End</option>
          </select>
        </label>
        <label title="Heightmaps can sometimes be incorrect, turning this off may fix rendering but be slower.">
          <input
            type="checkbox"
            checked={trustHeights}
            onChange={handleTrustHeightChange}
          />
          <span>Trust heightmaps</span>
        </label>
        <button
          title="Open a world that is potentially not in your saves directory"
          onClick={handleOpen}
        >
          Open folder...
        </button>
        <button
          title="Fly to (0,0) on the map"
          onClick={() => mapRef.current?.flyTo([0, 0])}
        >
          Fly to origin
        </button>
        {worldInfo && (
          <button
            title="Fly to spawn on the map"
            onClick={() =>
              mapRef.current?.flyTo([
                -worldInfo.level.Data.SpawnZ / 64,
                worldInfo.level.Data.SpawnX / 64,
              ])
            }
          >
            Fly to spawn
          </button>
        )}
      </Ribbon>
      <MapContainer
        ref={mapRef}
        crs={L.CRS.Simple}
        minZoom={2}
        className="map-container"
        center={[0, 0]}
        zoom={6}
        scrollWheelZoom={true}
      >
        {worldInfo && (
          <AnvilLayer
            heightmapMode={trustHeights ? "trust" : "calculate"}
            dimension={dimension}
            world={worldInfo}
          />
          // <SpawnNavigator world={worldDir} />
        )}
      </MapContainer>
    </div>
  );
}

export default App;
