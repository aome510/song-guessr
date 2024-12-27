import { useEffect, useRef } from "react";
import "./App.css";

function App() {
  const audioRef = useRef(new Audio());

  const fetchAudioUrl = async () => {
    try {
      const response = await fetch("http://localhost:8000/get");
      audioRef.current.src = await response.text();
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    fetchAudioUrl();
  }, []);

  return (
    <>
      <h1>Song Guesser</h1>
      <div>
        <audio ref={audioRef} autoPlay controls />
      </div>
      <div>
        <button onClick={fetchAudioUrl}>Re-generate</button>
      </div>
    </>
  );
}

export default App;
