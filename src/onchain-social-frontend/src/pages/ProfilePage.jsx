import React, { useEffect, useState } from 'react';
import backendActor from '../backend/backend';

function ProfilePage() {
  const [user, setUser] = useState(null);

  useEffect(() => {
    const loadUser = async () => {
      try {
        const userData = await backendActor.getUserProfile(); // 👈 Your backend function
        setUser(userData);
      } catch (err) {
        console.error("Failed to load user profile:", err);
      }
    };

    loadUser();
  }, []);

  if (!user) return <div>Loading profile...</div>;

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Welcome, {user.username}</h1>
      <p>Email: {user.email}</p>
      <p>Bio: {user.bio}</p>
    </div>
  );
}

export default ProfilePage;
