function __applyCommands(frame) {
    console.log(frame);

    for (const cmd of frame) {
        applyCommand(cmd);
    }
}

const nodes = new Map();

function applyCommand(cmd) {
    switch (cmd.type) {
        case "createNode": return createNode(cmd);
        case "deleteNode": return deleteNode(cmd);
        case "setText":    return setText(cmd);
        case "setStyle":   return setStyle(cmd);

        default:
            console.warn("Unknown command", cmd);
    }
}

function createNode({ node, kind }) {
    const e = document.createElement(kind);
    e.style.position = "absolute";
    e.style.boxSizing = "border-box";

    e.node = node;

    nodes.set(node, e);
    document.body.append(e)
}

function deleteNode({ node }) {
    const e = nodes.get(node);
    e.remove();
    nodes.delete(node);
}

function setText({ node, text }) {
    const e = nodes.get(node);
    e.textContent = text;
}

function setStyle({ node, key, value }) {
    const e = nodes.get(node);
    e.style[key] = value;
}
