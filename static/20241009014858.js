// Including Three.js for creating 3D animations
import * as THREE from 'https://cdn.jsdelivr.net/npm/three@0.136.0/build/three.module.js';

// Initialize the scene
const scene = new THREE.Scene();

// Adding a camera
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
camera.position.z = 5;

// Adding a renderer
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Adding a light source
const light = new THREE.PointLight(0xFFFFFF);
light.position.set(10, 0, 25);
scene.add(light);

// Create a function to add animated 3D objects to the scene
function add3DObjects() {
    // Create a cube and add it to the scene
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshPhongMaterial({ color: 0x44aa88 }); // greenish
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    // Animation function
    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
    }
    animate();
}

// Using Phaser.js for more interactive elements, possibly in 2D
import Phaser from 'https://cdn.jsdelivr.net/npm/phaser@3.55.2/dist/phaser.js';

const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    scene: {
        preload: preload,
        create: create
    }
};

const phaserGame = new Phaser.Game(config);

function preload() {
    this.load.image('background', '/static/background.jpg'); // assuming a background image is present
}

function create() {
    this.add.image(400, 300, 'background');
    this.add.text(400, 300, 'Emotive Journey', { font: '32px Arial', fill: '#ffffff' });

    // More phaser elements and animations can be added here
}

// Initialize the scene with 3D objects
add3DObjects();

// Function to handle window resizing
window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
});

// CSS styles can also be manipulated through JavaScript to make it savable as styles.css
const styles = `
body { 
    margin: 0; 
    overflow: hidden; 
    background: linear-gradient(45deg, #000428, #004e92);
} 
.header-container {
    text-align: center;
    color: #FFFFFF;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.5);
}
.nav-container {
    display: flex;
    justify-content: space-around;
    background-color: #1c1c1c;
    padding: 0.5rem 0;
}
.nav-link {
    text-decoration: none;
    color: #F0F0F0;
    font-weight: bold;
}
.unplayed_songs, .current-song, .charts-container {
    padding: 2rem;
}
.song, .user, .chart-container {
    background-color: rgba(255, 255, 255, 0.8);
    margin-bottom: 1rem;
    padding: 1rem;
    border-radius: 8px;
}
`;

// Saving style content to a downloadable CSS
const styleBlob = new Blob([styles], { type: 'text/css' });
const link = document.createElement("a");
link.href = URL.createObjectURL(styleBlob);
link.download = "styles.css";
link.innerText = "Download styles.css";
document.body.appendChild(link);

