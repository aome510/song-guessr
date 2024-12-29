import Search from "./Search.tsx";
import Game from "./Game.tsx";
import { Route, BrowserRouter, Routes } from "react-router-dom";
import "./App.css";

function App() {
  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Search />} />
          <Route path="/game" element={<Game />} />
        </Routes>
      </BrowserRouter>
    </>
  );
}

export default App;
