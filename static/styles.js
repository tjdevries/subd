// Importing necessary libraries for animations and interactivity
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Function to display a welcome message in the console
console.log('Welcome to AI Top of the Pops!');

// Function to animate the header with crazy unique fonts
const header = document.querySelector('.header');
header.style.fontWeight = 'bold';
header.style.fontFamily = 'UniqueFont, sans-serif'; // Import the unique font library here

// Function to create a dark, crazy visual effect on the page
const body = document.querySelector('body');
body.style.backgroundColor = '#121212'; // Dark background color

// Function to add animations and transitions to various elements on the page
const animateElements = () => {
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.style.transition = 'transform 0.5s ease';
        link.addEventListener('mouseover', () => {
            link.style.transform = 'scale(1.1)';
        });
        link.addEventListener('mouseout', () => {
            link.style.transform = 'scale(1)';
        });
    });

    const songs = document.querySelectorAll('.song');
    songs.forEach(song => {
        song.style.transition = 'transform 0.5s ease';
        song.addEventListener('mouseover', () => {
            song.style.transform = 'rotate(5deg)';
        });
        song.addEventListener('mouseout', () => {
            song.style.transform = 'rotate(0deg)';
        });
    });

    const images = document.querySelectorAll('.image');
    images.forEach(image => {
        image.style.transition = 'opacity 0.5s ease';
        image.addEventListener('mouseover', () => {
            image.style.opacity = 0.7;
        });
        image.addEventListener('mouseout', () => {
            image.style.opacity = 1;
        });
    });
};

// Call the animateElements function to add animations and transitions
animateElements();