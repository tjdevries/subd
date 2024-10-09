// Utilizing Three.js to create a 3D animated sailing scene inspired by the poetry given.

// Import Three.js library
import * as THREE from 'three';

// Create the scene
const scene = new THREE.Scene();

// Create a camera, which determines what we'll see when we render the scene
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);

// Create a renderer and attach it to our document
const renderer = new THREE.WebGLRenderer({antialias: true});
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add orbit controls (optional)
const controls = new THREE.OrbitControls(camera, renderer.domElement);

// Setting up the ocean using PlaneGeometry
const oceanGeometry = new THREE.PlaneGeometry(1000, 1000, 100, 100);
const oceanMaterial = new THREE.MeshBasicMaterial({color: 0x1E90FF, side: THREE.DoubleSide, wireframe: true}); // Use wireframe for fun effect
const ocean = new THREE.Mesh(oceanGeometry, oceanMaterial);
ocean.rotation.x = Math.PI / 2;
scene.add(ocean);

// Creating a sailboat
const boatGeometry = new THREE.ConeGeometry(5, 20, 32);
const boatMaterial = new THREE.MeshBasicMaterial({color: 0x8B4513});
const boat = new THREE.Mesh(boatGeometry, boatMaterial);
boat.position.y = 10;
scene.add(boat);

// Add ambient light
const ambientLight = new THREE.AmbientLight(0xFFFFFF, 0.5);
scene.add(ambientLight);

// Add directional light
const directionalLight = new THREE.DirectionalLight(0xFFFFFF, 1);
directionalLight.position.set(1, 1, 1).normalize();
scene.add(directionalLight);

// Position the camera
camera.position.z = 50;

// Animation loop
function animate() {
    requestAnimationFrame(animate);

    // Add motion to simulate sailing through code
    ocean.rotation.z += 0.005;
    boat.position.x += Math.sin(ocean.rotation.z) * 0.1;
    boat.rotation.z += 0.01;

    // Render scene with camera
    renderer.render(scene, camera);
}

animate();