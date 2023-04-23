import { useRef, useState } from "react";
import "./App.css";
import { open } from "@tauri-apps/api/dialog";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

import { MapContainer } from "react-leaflet";
import { AnvilLayer } from "./AnvilLayer";
import { Ribbon } from "./Ribbon";
import { savesDir } from "./openWorld";
import { WorldInfo, WorldSelect, useWorldSelect } from "./WorldSelect";

function App() {
  const [worldDir, setWorldDir] = useState<string | undefined>();
  const [trustHeights, setTrustHeights] = useState(true);
  const [dimension, setDimension] = useState("overworld");
  const worlds = useWorldSelect();
  const mapRef = useRef<L.Map | null>(null);

  async function handleOpen() {
    const saves = await savesDir();

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
  // TODO: Open arb directory for world.
  // TODO: Show coordinates.
  // TODO: Distingush broken vs missing regions.
  // TODO: Fix region loading indication not disappearing properly.
  // TODO: Search for item! eg specific enchantment or name
  // TODO: Add players! Show inventory, enderchest, acheivements.
  // TODO: Fly to player. Could be any dimension.
  // TODO: Search for block/entity
  // TODO: Extract icon.png.
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
      </Ribbon>
      <MapContainer
        ref={mapRef}
        crs={L.CRS.Simple}
        minZoom={2}
        className="map-container"
        center={[0, 0]}
        zoom={6}
        scrollWheelZoom={true}
        // @ts-ignore
        loadingControl={true}
      >
        {worldDir && (
          <AnvilLayer
            heightmapMode={trustHeights ? "trust" : "calculate"}
            dimension={dimension}
            worldDir={worldDir}
          />
        )}
      </MapContainer>
    </div>
  );
}

export default App;
