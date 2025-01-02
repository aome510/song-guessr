import Search from "./Search.tsx";
import Game from "./Game.tsx";
import { Route, BrowserRouter, Routes } from "react-router-dom";
import "./App.css";

const NotFound = () => <h1>404 - Page Not Found</h1>;

function App() {
  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Search />} />
          <Route path="/game/:id" element={<Game />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
    </>
  );
}

export default App;
