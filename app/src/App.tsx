import { useRef, useState } from "react";
import "./App.css";
import { open } from "@tauri-apps/api/dialog";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

import { MapContainer } from "react-leaflet";
import { AnvilLayer } from "./AnvilLayer";
import { Ribbon } from "./Ribbon";
import { savesDir } from "./openWorld";

function App() {
  const [worldDir, setWorldDir] = useState<string | undefined>();
  const [calcHeights, setCalcHeights] = useState(false);
  const [dimension, setDimension] = useState("overworld");
  const mapRef = useRef<L.Map | null>(null);

  async function handleOpen() {
    const saves = await savesDir();

    const selected = await open({
      multiple: false,
      directory: true,
      defaultPath: saves ?? undefined,
    });

    if (selected && !Array.isArray(selected)) {
      setWorldDir(selected);
    }
  }

  function handleHeightmapModeChange(ev: React.ChangeEvent<HTMLInputElement>) {
    setCalcHeights((prev) => !prev);
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
  return (
    <div className="container">
      <Ribbon>
        <button onClick={handleOpen}>Open save...</button>
        <label>
          <select onChange={handleDimensionChange}>
            <option value="overworld">Overworld</option>
            <option value="nether">Nether</option>
            <option value="end">The End</option>
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            checked={calcHeights}
            onChange={handleHeightmapModeChange}
          />
          Recalculate Heightmaps
        </label>
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
            heightmapMode={calcHeights ? "calculate" : "trust"}
            dimension={dimension}
            worldDir={worldDir}
          />
        )}
      </MapContainer>
    </div>
  );
}

export default App;
