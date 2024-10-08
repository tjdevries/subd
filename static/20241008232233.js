// We will use Vanilla JavaScript to add animations and make the page interactive and fun.

// Function to animate the header text
function animateHeader() {
    const header = document.querySelector('.header');
    header.classList.add('animated-header');
}

// Function to animate navigation links on hover
function animateNavLinks() {
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('mouseover', () => {
            link.classList.add('nav-hover');
        });
        link.addEventListener('mouseout', () => {
            link.classList.remove('nav-hover');
        });
    });
}

// Function to animate "Songs in Playlist"
function animateUnplayedSongs() {
    const unplayedSongs = document.querySelectorAll('.unplayed_song');
    unplayedSongs.forEach((song, index) => {
        song.style.animationDelay = `${index * 0.1}s`;
        song.classList.add('fade-in');
    });
}

// Function to add interactivity to image voting
function addImageVotingListeners() {
    const imageVotes = document.querySelectorAll('.image_voting');
    imageVotes.forEach(vote => {
        vote.style.cursor = 'pointer';
        vote.addEventListener('click', function() {
            alert('You voted!');
        });
    });
}

// Function to make video play on hover
function enableVideoHover() {
    const videoElements = document.querySelectorAll('.video video');
    videoElements.forEach(video => {
        video.addEventListener('mouseover', () => {
            video.play();
        });
        video.addEventListener('mouseout', () => {
            video.pause();
        });
    });
}

// Initialize all animations and interactive elements when the document is loaded
function initializePage() {
    animateHeader();
    animateNavLinks();
    animateUnplayedSongs();
    addImageVotingListeners();
    enableVideoHover();
}

document.addEventListener('DOMContentLoaded', initializePage);

// CSS animation classes and other styles will be defined in the styles.css file.