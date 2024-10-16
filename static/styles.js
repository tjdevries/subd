import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize the scene for Three.js animations
console.log('Initializing Three.js scene...');
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add a rotating cube to the scene to symbolize the spinning thoughts about pigs
console.log('Creating a rotating cube in the scene...');
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;
function animateCube() {
    requestAnimationFrame(animateCube);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
}
console.log('Starting cube animation...');
animateCube();

// Adding D3.js visualization for pig statistics
console.log('Loading D3 Visualization for pig statistics...');
const pigData = [{ condition: 'Present', number: 10 }, { condition: 'Absent', number: 90 }];
const svg = D3.select("body").append("svg").attr("width", 300).attr("height", 200);
svg.selectAll("rect")
   .data(pigData)
   .enter()
   .append("rect")
   .attr("x", (d, i) => i * 50)
   .attr("y", d => 200 - d.number * 2)
   .attr("width", 40)
   .attr("height", d => d.number * 2)
   .attr("fill", "blue");

// Initialize mermaid.js for flowcharts
console.log('Initializing Mermaid.js for flowchart animation...');
mermaid.initialize({ startOnLoad: true });
const graphDefinition = `
  graph TB
    A(Not enough pigs) --> B(Empty pens)
    A --> C(Quiet days)
    B --> D(April's gray without noise)
    C --> D
    D -->|Memory's maze| E[/Lost/]
`;
mermaid.render('mermaidChart', graphDefinition, function(svgCode) {
    const div = document.createElement('div');
    div.innerHTML = svgCode;
    document.body.appendChild(div);
});

console.log('Finished setting up animations and visualizations!');