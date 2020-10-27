import { force_init } from "./pkg/anvil_wasm_bg.wasm";

export * from "./pkg/anvil_wasm_bg.js";

force_init();

import { TileRenderer } from "./pkg/anvil_wasm";

const tileRenderer = TileRenderer.new();

onmessage = e => {
    const { region, fileName } = e.data;
    const region_arr = new Uint8Array(region);
    const data = tileRenderer.render(region_arr);

    postMessage({
        fileName,
        data: data.buffer,
    }, [data.buffer]);
}