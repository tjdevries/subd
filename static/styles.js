document.addEventListener('DOMContentLoaded', () => {
    // Function to add animation class to elements
    const animateElement = (element, animationName) => {
        element.classList.add('animated', animationName);
        element.addEventListener('animationend', () => {
            element.classList.remove('animated', animationName);
        });
    };

    // Animate header on page load
    const header = document.querySelector('.header-container');
    animateElement(header, 'bounceInDown');

    // Animate navigation links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach((link, index) => {
        setTimeout(() => {
            animateElement(link, 'fadeInUp');
        }, index * 150);
    });

    // Animate song titles in unplayed songs section
    const unplayedSongs = document.querySelectorAll('.unplayed_song a');
    unplayedSongs.forEach((song, index) => {
        setTimeout(() => {
            animateElement(song, 'zoomIn');
        }, index * 100);
    });

    // Animate current song section
    const currentSongSection = document.querySelector('.current-song-info');
    if (currentSongSection) {
        setTimeout(() => {
            animateElement(currentSongSection, 'fadeInLeft');
        }, 600);
    }

    // Add hover effect to images
    const images = document.querySelectorAll('.ai_song_image img');
    images.forEach(img => {
        img.addEventListener('mouseenter', () => {
            animateElement(img, 'pulse');
        });
    });

    // Add hover effect to videos
    const videos = document.querySelectorAll('.video video');
    videos.forEach(video => {
        video.addEventListener('mouseenter', () => {
            animateElement(video, 'rubberBand');
        });
    });

    // Add animations to grid songs
    const gridSongs = document.querySelectorAll('.grid-container .song');
    gridSongs.forEach((song, index) => {
        setTimeout(() => {
            animateElement(song, 'fadeIn');
        }, index * 100);
    });

    // Animate user section
    const users = document.querySelectorAll('.users-container .username, .total-songs, .average-score');
    users.forEach((user, index) => {
        setTimeout(() => {
            animateElement(user, 'lightSpeedIn');
        }, index * 200);
    });

    // Animate charts section
    const charts = document.querySelectorAll('.songs-container .song-container');
    charts.forEach((chart, index) => {
        setTimeout(() => {
            animateElement(chart, 'flipInX');
        }, index * 150);
    });
});