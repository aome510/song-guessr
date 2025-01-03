import { useParams } from "react-router-dom";

function Room() {
  const { id } = useParams();
  return (
    <div>
      <h1>Room {id}</h1>
    </div>
  );
}

export default Room;
