import init, * as wasm from "orlice";

const inputSavefile = document.getElementById("input_savefile");

async function run() {
    await init();
    //wasm.greet();
}

inputSavefile.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    const buffer = await file.arrayBuffer();
    const bytes = new Uint8Array(buffer);

    // Pass to Rust
    // https://rustwasm.app/en/learn/file-processing
    try {
        const data = await wasm.generate_image(bytes);
        const blob = new Blob([data], { type: 'image/png' });
        const url = URL.createObjectURL(blob);

        const a = document.createElement('a');
        a.href = url;
        a.download = 'map.png';
        a.click();
        URL.revokeObjectURL(url);
    } catch (err) {
        console.log(err.message, err.code); // structured ✅
    }
});

run();
