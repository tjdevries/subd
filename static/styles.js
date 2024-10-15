import * as THREE from 'three';
import * as d3 from 'd3';
import * as mermaid from 'mermaid';

// Initializing Mermaid for the chart rendering
mermaid.initialize({startOnLoad:true});

console.log("Mermaid initialized.");

// Summarized Content of the Song
const songContent = {
    theme: "God is a Song and the Pope is the DJ",
    verses: [
        "God is a song on the radio high - Spinning the world make it touch the sky",
        "Saints on the turntables spinning the vinyl",
        "Glory in the bass Jesus in the keys"
    ],
    chorus: "Heavenly tunes making every soul sway",
    bridge: "Divine in the melody pulpit in the song"
};

console.log("Song content loaded:", songContent);

// Initialize scene for three.js
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();

renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);
console.log("Three.js scene initialized.");

camera.position.z = 5;

// Create 3D objects (e.g., rotating vinyl record to symbolize the DJ theme)
const geometry = new THREE.CircleGeometry(2, 64);
const material = new THREE.MeshBasicMaterial({color: 0xaaaaaa});
const record = new THREE.Mesh(geometry, material);

scene.add(record);
console.log("Vinyl record object added to scene.");

// Animation loop
function animate() {
    requestAnimationFrame(animate);

    // Rotate the record for animation
    record.rotation.z += 0.01;

    renderer.render(scene, camera);
}

animate();
console.log("Animation loop started.");

// Adding D3.js chart for song statistics
const data = [ {name: "Heavenly Beats", value: 10}, {name: "Holy Streets", value: 15}, {name: "Cosmic Rave", value: 20} ];

const svg = d3.select("body").append("svg")
    .attr("width", 300)
    .attr("height", 200);
console.log("D3.js SVG initialized.");

svg.selectAll("rect")
  .data(data)
  .enter().append("rect")
    .attr("x", (d, i) => i * (300 / data.length))
    .attr("y", d => 200 - d.value * 10)
    .attr("width", 40)
    .attr("height", d => d.value * 10)
    .attr("fill", "orange");

svg.selectAll("text")
  .data(data)
  .enter().append("text")
    .text(d => d.name)
    .attr("x", (d, i) => i * (300 / data.length) + 10)
    .attr("y", d => 200 - (d.value * 10) - 3)
    .attr("fill", "white");

console.log("D3.js chart elements added and configured.");

// Mermaid diagrams for visual poem
const mermaidGraph = `graph LR
A((God's Song)) -->|holy beats| B(Pope's DJ Set)
B --> C{Heavenly Playlists}
C -->|gospel| D[Hearts Dance]
C -->|symphony| E[Souls in Sway]
D --> F[Divine Melodies]
E --> F`;

mermaid.render('theGraph', mermaidGraph, (svgCode, bindFunctions) => {
    document.getElementById('mermaid').innerHTML = svgCode;
    console.log("Mermaid graph rendered:", svgCode);
});

console.log("Script execution complete.");