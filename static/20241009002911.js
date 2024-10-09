// Import Three.js
import * as THREE from 'three';

// Create a scene
const scene = new THREE.Scene();

// Set up camera
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
camera.position.z = 5;

// Render to canvas
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Load textures
const textureLoader = new THREE.TextureLoader();
const auroraTexture = textureLoader.load('/textures/aurora.jpg');

// Create a plane geometry
const geometry = new THREE.PlaneGeometry(5, 5, 32, 32);
const material = new THREE.MeshStandardMaterial({ map: auroraTexture });
const plane = new THREE.Mesh(geometry, material);
scene.add(plane);

// Add lights
const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
scene.add(ambientLight);

const pointLight = new THREE.PointLight(0xffffff, 1);
pointLight.position.set(5, 5, 5);
scene.add(pointLight);

// Animation function
function animate() {
    requestAnimationFrame(animate);

    // Rotate the plane to create an aurora effect
    plane.rotation.x += 0.005;
    plane.rotation.y += 0.01;

    renderer.render(scene, camera);
}

animate();