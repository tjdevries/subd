// Assuming three.js is already loaded in the project

// Create the scene
const scene = new THREE.Scene();

// Create a camera, which determines what we'll see when we render the scene
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
camera.position.z = 5;

// Create a renderer
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add some pizza bagel animations for fun
const geometry = new THREE.TorusGeometry(3, 1, 16, 100);
const material = new THREE.MeshBasicMaterial({ color: 0xffff00, wireframe: true });
const torus = new THREE.Mesh(geometry, material);

// Add the torus (bagel) to the scene
scene.add(torus);

// Animation loop
function animate() {
    requestAnimationFrame(animate);

    // Rotate the pizza bagel
    torus.rotation.x += 0.01;
    torus.rotation.y += 0.01;

    // Render the scene from the perspective of the camera
    renderer.render(scene, camera);
}

// Set ambient light to mimic morning light
const ambientLight = new THREE.AmbientLight(0x404040); 
scene.add(ambientLight);

// Oven light
const pointLight = new THREE.PointLight(0xffffff);
pointLight.position.set(5, 5, 5);
scene.add(pointLight);

// Call the animation loop
animate();

// Handle responsive design
window.addEventListener('resize', () => {
    const width = window.innerWidth;
    const height = window.innerHeight;

    renderer.setSize(width, height);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
});