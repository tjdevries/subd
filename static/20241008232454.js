document.addEventListener('DOMContentLoaded', () => {
    // Animation settings
    const animateCSS = (element, animation, prefix = 'animate__') =>
        // Return a Promise which resolves when the animation ends
        new Promise((resolve, reject) => {
            const animationName = `${prefix}${animation}`;
            const node = document.querySelector(element);

            node.classList.add(`${prefix}animated`, animationName);

            // When the animation ends, we remove the class and resolve the Promise
            const handleAnimationEnd = event => {
                event.stopPropagation();
                node.classList.remove(`${prefix}animated`, animationName);
                resolve('Animation ended');
            };

            node.addEventListener('animationend', handleAnimationEnd, { once: true });
        });

    // Apply animations
    animateCSS('.header-container', 'bounceInDown');
    animateCSS('.nav-container', 'fadeInLeft');
    animateCSS('.song', 'fadeInUp');
    animateCSS('.video', 'zoomIn');

    // Adding hover effects to nav links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('mouseover', () => animateCSS(link, 'pulse'));
    });

    // Animate image set
    let currentIndex = 0;
    const images = document.querySelectorAll('.ai_song_image');

    const showNextImage = () => {
        images[currentIndex].classList.remove('active');
        currentIndex = (currentIndex + 1) % images.length;
        images[currentIndex].classList.add('active');
        animateCSS(images[currentIndex], 'fadeIn');
    };

    setInterval(showNextImage, 3000);

    // Random animation for titles
    const songTitles = document.querySelectorAll('.song .title');
    const animations = ['bounce', 'flash', 'pulse', 'rubberBand', 'shakeX', 'shakeY', 'swing', 'tada', 'wobble', 'jello', 'heartBeat'];

    songTitles.forEach(title => {
        title.addEventListener('click', () => {
            const randomAnimation = animations[Math.floor(Math.random() * animations.length)];
            animateCSS(title, randomAnimation);
        });
    });
});