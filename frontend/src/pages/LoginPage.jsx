import { LoginForm } from "@/components/login-form";
import { userService } from "@/services/userService";
import { useState } from "react";

export default function LoginPage() {
  const [error, setError] = useState('')

  const handleLogin = async (email, password) => {
    try {
      const response = await userService.login({ email, password });
      console.log("Success:", response.data);
    } catch (err) {
      console.error("Login failed:", err);
      setError("Incorrect email or password!");
    }
  };

  return (
    <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
      <div className="w-full max-w-sm">
        <LoginForm onSubmit={handleLogin} error={error}/>
      </div>
    </div>
  )
}
