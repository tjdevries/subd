// JavaScript Code utilizing Three.js and other creative animations

// Initialize the scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add ambient light
const ambientLight = new THREE.AmbientLight(0x404040, 2); // Soft white light
scene.add(ambientLight);

// Add a point light
const pointLight = new THREE.PointLight(0xffffff, 1, 100);
pointLight.position.set(50, 50, 50);
scene.add(pointLight);

// Create and add a rotating cube with music visualizer style
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshStandardMaterial({ color: 0x0077ff });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

// Set camera position
camera.position.z = 5;

// Add particles using Three.js
const particles = new THREE.BufferGeometry();
const particleCount = 5000;
const posArray = new Float32Array(particleCount * 3);

for (let i = 0; i < particleCount * 3; i++) {
    posArray[i] = (Math.random() - 0.5) * 10;
}

particles.setAttribute('position', new THREE.BufferAttribute(posArray, 3));
const particleMaterial = new THREE.PointsMaterial({ size: 0.05, color: 0xff0000 });
const particleMesh = new THREE.Points(particles, particleMaterial);
scene.add(particleMesh);

// Animation loop for rotation and particle movement
function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    particleMesh.rotation.y += 0.002;
    renderer.render(scene, camera);
}

animate();

// Add interactive hover animations on nav links
const navLinks = document.querySelectorAll('.nav-link');
navLinks.forEach(link => {
    link.addEventListener('mouseenter', () => {
        link.style.transition = 'all 0.3s ease-in-out';
        link.style.transform = 'scale(1.2)';
        link.style.color = '#ff4500';
    });

    link.addEventListener('mouseleave', () => {
        link.style.transform = 'scale(1)';
        link.style.color = '';
    });
});

// Optional: Song images bouncing effect
const songImages = document.querySelectorAll('.ai_song_image img');
songImages.forEach(image => {
    image.addEventListener('click', () => {
        image.style.transition = 'transform 0.5s ease';
        image.style.transform = 'translateY(-20px)';
        setTimeout(() => {
            image.style.transform = 'translateY(0)';
        }, 500);
    });
});