import init, * as wasm from "orlice";

const inputSavefile = document.getElementById("input_savefile");
const btnAddNation = document.getElementById("btnAddNation");
const showAllies = document.getElementById("showAllies");
const blendAllies = document.getElementById("blendAllies");
const showSubjects = document.getElementById("showSubjects");
const blendSubjects = document.getElementById("blendSubjects");
const subjectCol = document.getElementById("subjectCol");
var optionsArray = ["BOH", "SWE"];

async function run() {
    await init();
    //wasm.greet();
}

showAllies.addEventListener('click', () => {
    blendAllies.disabled = !showAllies.checked;

    if (blendAllies.disabled) {
        blendAllies.checked = false;
    }

});

showSubjects.addEventListener('click', () => {
    blendSubjects.disabled = !showSubjects.checked;
    subjectCol.disabled = !showSubjects.checked;

    if (blendSubjects.disabled) {
        blendSubjects.checked = false;
        subjectCol.checked = false;
    }
});

inputSavefile.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    const buffer = await file.arrayBuffer();
    const bytes = new Uint8Array(buffer);

    try {
        optionsArray = wasm.get_nation_tags(bytes);
        btnAddNation.disabled = false;
    } catch (err) {
        console.log(err.message, err.code);
    }
});

function createSelect() {
    const select = document.createElement("select");

    optionsArray.forEach(item => {
        const option = document.createElement("option");
        option.value = item;
        option.textContent = item;
        select.appendChild(option);
    });

    return select;
}

btnAddNation.addEventListener('click', () => {
    const container = document.getElementById("inputsContainer");

    const row = document.createElement("div");
    row.className = "input-row";

    const select = createSelect();

    const removeBtn = document.createElement("button");
    removeBtn.textContent = "X";
    removeBtn.onclick = () => row.remove();

    row.appendChild(select);
    row.appendChild(removeBtn);

    container.appendChild(row);
});

btnSubmitForm.addEventListener('click', async () => {
    var file = inputSavefile.files[0];
    const buffer = await file.arrayBuffer();
    const bytes = new Uint8Array(buffer);

    const imageOptions = [showAllies.checked, blendAllies.checked, showSubjects.checked, blendAllies.checked, subjectCol.checked];
    console.log(imageOptions);

    const countryTags = Array.from(document.querySelectorAll("#inputsContainer select"))
    .map(sel => sel.value);
    console.log(countryTags);

    try {
        const data = await wasm.generate_image(bytes, countryTags, imageOptions);
        const blob = new Blob([data], { type: 'image/png' });
        const url = URL.createObjectURL(blob);

        const a = document.createElement('a');
        a.href = url;
        a.download = 'map.png';
        a.click();
        URL.revokeObjectURL(url);
    } catch (err) {
        console.log(err.message, err.code);
    }
});

run();
