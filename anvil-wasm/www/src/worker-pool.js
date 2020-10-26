
export default class WorkerPool {
    constructor(count, onMessage) {
        this.workers = [];
        this.messageCount = 0;
        for (let i = 0; i < count; i++) {
            const worker = new Worker('./worker.bundle.js');
            worker.onmessage = onMessage;
            worker.onerror = console.error;
            this.workers.push(worker);
        }
    }

    postMessage(data) {
        const i = this.messageCount % this.workers.length;
        this.messageCount++;

        this.workers[i].postMessage(data);
    }
}
