import { useEffect, useState } from "react";
import "./App.css";

function App() {
  const [currentTrackUrl, setCurrentTrackUrl] = useState("");

  useEffect(() => {
    const fetchAudioUrl = async () => {
      try {
        const response = await fetch("http://localhost:8000/get");
        const text = await response.text();
        setCurrentTrackUrl(text);
      } catch (err) {
        console.error(err);
      }
    };

    fetchAudioUrl();
  }, []);

  return (
    <>
      <h1>Song Guesser</h1>
      <audio src={currentTrackUrl} controls />
    </>
  );
}

export default App;
