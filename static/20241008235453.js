// JavaScript code to add animations and fun effects to the HTML page

// Check if the document is fully loaded
window.addEventListener('DOMContentLoaded', (event) => {
    // Add Three.js Scene
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();

    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Create a cube
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    // Animation loop for the cube
    function animate() {
        requestAnimationFrame(animate);

        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;

        renderer.render(scene, camera);
    }
    animate();

    // Add some fun animations to the header text
    const header = document.querySelector('.header');
    header.style.position = 'relative';
    header.style.animation = 'bounce 2s infinite';

    // Define keyframes for the bounce animation
    const style = document.createElement('style');
    style.type = 'text/css';
    style.innerHTML = `
        @keyframes bounce {
            0%, 20%, 50%, 80%, 100% {
                transform: translateY(0);
            }
            40% {
                transform: translateY(-30px);
            }
            60% {
                transform: translateY(-15px);
            }
        }
    `;
    document.getElementsByTagName('head')[0].appendChild(style);

    // Adding hover effect on navigation links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('mouseover', () => {
            link.style.color = 'red';
            link.style.transition = 'color 0.3s';
            link.style.textShadow = '0 0 5px rgba(255, 255, 255, 0.9)';
        });
        link.addEventListener('mouseout', () => {
            link.style.color = '';
            link.style.textShadow = '';
        });
    });

    // Add parallax scrolling
    window.addEventListener('scroll', () => {
        const offset = window.pageYOffset;
        scene.position.y = offset * -0.1;
    });

    // Phaser game initialization
    const config = {
        type: Phaser.AUTO,
        width: 800,
        height: 600,
        scene: {
            preload: preload,
            create: create
        }
    };

    const game = new Phaser.Game(config);

    function preload() {
        this.load.image('star', '/path/to/star.png');
    }

    function create() {
        this.add.image(400, 300, 'star');
    }
});