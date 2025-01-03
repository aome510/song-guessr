import { Route, BrowserRouter, Routes } from "react-router-dom";
import "./App.css";
import Room from "./Room.tsx";
import HomePage from "./HomePage.tsx";

const NotFound = () => <h1>404 - Page Not Found</h1>;

function App() {
  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/room/:id" element={<Room />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
    </>
  );
}

export default App;
