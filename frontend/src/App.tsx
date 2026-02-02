import { useState } from 'react'
import LoginForm from './components/LoginForm'
import PasswordChangeForm from './components/PasswordChangeForm'
import './index.css'

function App() {
  const [user, setUser] = useState<string | null>(null);
  const [showPasswordChange, setShowPasswordChange] = useState(false);

  const handleLoginSuccess = (username: string) => {
    setUser(username);
  };

  const handleLogout = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      setUser(null);
      setShowPasswordChange(false);
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  const handlePasswordChange = () => {
    setShowPasswordChange(true);
  };

  const handlePasswordChangeSuccess = () => {
    setShowPasswordChange(false);
    handleLogout(); // Log out after successful password change
  };

  const handleCancelPasswordChange = () => {
    setShowPasswordChange(false);
  };

  if (!user) {
    return <LoginForm onLoginSuccess={handleLoginSuccess} />;
  }

  if (showPasswordChange) {
    return <PasswordChangeForm onPasswordChangeSuccess={handlePasswordChangeSuccess} onCancel={handleCancelPasswordChange} />;
  }

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center p-4">
      <div className="flex items-center text-center justify-center "> 
        <img src="/img/katalyst_basic.png" alt="Logo" className="w-32 h-32 mx-auto mb-4" />
      </div>
      <div className="bg-white p-8 rounded-lg shadow-md w-full max-w-md text-center">
        <h1 className="text-2xl font-bold mb-4 text-gray-800">Welcome, {user}!</h1>
        <p className="mb-6 text-gray-600">You are securely logged in.</p>
        
        <div className="space-y-4">
          <button
            onClick={handlePasswordChange}
            className="w-full bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline transition duration-150"
          >
            Change Password
          </button>
          
          <button
            onClick={handleLogout}
            className="w-full bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline transition duration-150"
          >
            Sign Out
          </button>
        </div>
      </div>
    </div>
  )
}

export default App
