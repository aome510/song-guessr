import { useNavigate } from "react-router-dom";
import UserForm from "./UserForm";
import { getUserData, post } from "./utils";

function HomePage() {
  const userData = getUserData();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const response = await post("api/room", {
        user_id: userData?.id,
      });
      const data = await response.json();
      navigate(`/room/${data.room_id}`);
    } catch (err) {
      console.error(err);
    }
  };

  if (userData === null) {
    return <UserForm />;
  }

  return (
    <div>
      <h1>Song Guessr</h1>
      <h2>Welcome, {userData.name}</h2>
      <form onSubmit={handleSubmit}>
        <button type="submit">Create a new room</button>
      </form>
    </div>
  );
}

export default HomePage;
