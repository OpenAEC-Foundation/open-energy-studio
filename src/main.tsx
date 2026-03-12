import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// Apply stored theme before first render to prevent flash
{
  let stored = localStorage.getItem('energy-theme') || 'dark';
  // Migrate removed themes to dark
  if (stored === 'blue' || stored === 'openaec') {
    stored = 'dark';
    localStorage.setItem('energy-theme', 'dark');
  }
  const effective = stored === 'system'
    ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
    : stored;
  document.documentElement.dataset.theme = effective;
}

// Disable browser context menu in production
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
