import L from 'leaflet';
import { useLeafletContext } from "@react-leaflet/core";
import { useEffect } from 'react';
import { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

import { invoke } from '@tauri-apps/api/tauri';

interface AnvilLayerProps {
    /// Base path of a Minecraft world. This should be the directory containing
    /// the entire world, so the DIM1, DIM-1 and region directories are within
    /// this directory.
    worldDir: string,
}

export function AnvilLayer({ worldDir }: AnvilLayerProps) {
    const context = useLeafletContext();

    useEffect(() => {
        const container = context.layerContainer || context.map;
        const layer = make_layer({ worldDir }, {
            minNativeZoom: 6,
            maxNativeZoom: 6,
            tileSize: 512,
            noWrap: true,
        });
        let unlisten: UnlistenFn | null = null;
        let destructed = false;

        listen<TileResponse>('tile_rendered', (event) => {
            layer.handleTileResponse(event.payload)
        }).then((unlistenFn) => {
            // Handle the race condition where the component is unmounted before
            // this runs. If the destructor has already ran, it won't call
            // unlisten, in which case we call it here.
            if (destructed) {
                unlistenFn();
            }
            // if it hasn't run, we save this for the destructor to call.
            unlisten = unlistenFn;
        })

        container.addLayer(layer);

        return () => {
            destructed = true;
            container.removeLayer(layer);
            unlisten && unlisten();
        };
    }, [worldDir]);

    return null;
}

function make_layer(args: AnvilLayerInnerArgs, leafletOpts: Object): AnvilLayerInner & L.Layer {
    // Leaflet types don't figure out the constructor args. These are the same
    // as get passed to the initialize function in the extend function below.
    // This makes a TS safe wrapper around the problem.

    // @ts-ignore
    return new _AnvilLayerInner({ ...args, ...leafletOpts });
}

interface AnvilLayerInnerArgs {
    worldDir: string,
}

interface AnvilLayerInner {
    handleTileResponse: (resp: TileResponse) => void;
}

type TileResponse = TileRender | TileError;

interface TileRender {
    kind: "render",
    rx: number,
    rz: number,
    dimension: string,
    basePath: string,
    imageData: string,
}

interface TileError {
    kind: "error",
    rx: number,
    rz: number,
    dimension: string,
    basePath: string,
    message: string,
}

interface Callback {
    done: (error: Error | null, tile: HTMLImageElement) => void,
    tile: HTMLImageElement,
    /// Has this tile already been requested before?
    cached: boolean,
}

type CallbackMap = Map<string, Callback>;

const _AnvilLayerInner = L.GridLayer.extend({
    initialize: function (args: AnvilLayerInnerArgs) {
        // @ts-ignore
        L.GridLayer.prototype.initialize.call(this, args);

        this.args = args;
        this.callbacks = new Map();
        this.handleTileResponse = (resp: TileResponse) => {
            if (resp.kind === "render") {
                const callbacks: CallbackMap = this.callbacks;
                const val = callbacks.get(`${resp.rx},${resp.rz}`);

                if (!val) {
                    console.log("no key:", resp);
                    return;
                }
                const { done, tile, cached } = val;

                if (cached) {
                    done(null, tile);
                    return;
                }

                tile.src = "data:image/png;base64," + resp.imageData;
                const key = `${resp.rx},${resp.rz}`;
                callbacks.set(key, { done, tile, cached: true });
                console.log("Tile element", tile);
                done(null, tile);
            } else {
                console.error(resp);
            }
        };
    },
    createTile: function (coords: any, done: any) {
        const args: AnvilLayerInnerArgs = this.args;
        const callbacks: CallbackMap = this.callbacks;

        // in minecraft x/z is the floor, but in leaflet x/y is.
        const req = { rx: coords.x, rz: coords.y, dimension: "overworld", worldDir: args.worldDir };

        const key = `${req.rx},${req.rz}`;
        const val = callbacks.get(key)

        if (val) {
            // request already been made.
            return val.tile;
        } else {
            // FIXME: Version of safari/webkit that Tauri uses does not support
            // crisp rendering on canvas elements, only img elements. Can we
            // change to img elements?
            var tile = L.DomUtil.create('img', 'leaflet-tile');
            var size = this.getTileSize();
            (<any>tile).width = size.x;
            (<any>tile).height = size.y;

            callbacks.set(key, { done, tile, cached: false });
            invoke('render_tile', req).then(() => {
                console.log("Invoked render-tile:", coords, req);
            })

            return tile;
        }
    }
});


async function fromB64(data: string): Promise<Uint8ClampedArray> {
    const dataUrl = "data:application/octet-binary;base64," + data;

    // Use fetch to convert the base64.
    const resp = await fetch(dataUrl);
    const buf = await resp.arrayBuffer();
    return new Uint8ClampedArray(buf);
}