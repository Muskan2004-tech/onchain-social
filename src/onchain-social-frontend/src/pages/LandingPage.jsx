import React from 'react';
import { useNavigate } from 'react-router-dom';
import './LandingPage.css';

const LandingPage = () => {
  const navigate = useNavigate();

  return (
    <div className="landing-container">
      <div className="hero">
        <h1 className="animated-title">Welcome to OnChain Social</h1>
        <p className="subtitle">Your decentralized social experience</p>
        <div className="buttons">
          <button className="btn login-btn" onClick={() => navigate('/login')}>Login</button>
          <button className="btn register-btn" onClick={() => navigate('/register')}>Register</button>
        </div>
      </div>
    </div>
  );
};

export default LandingPage;
