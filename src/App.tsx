import { Route, BrowserRouter, Routes } from "react-router-dom";
import Room from "./Room.tsx";
import HomePage from "./HomePage.tsx";
import { ChakraProvider, defaultSystem, Heading } from "@chakra-ui/react";

const NotFound = () => <Heading size="6xl">404 - Page Not Found</Heading>;

function App() {
  return (
    <>
      <ChakraProvider value={defaultSystem}>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/room/:id" element={<Room />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </BrowserRouter>
      </ChakraProvider>
    </>
  );
}

export default App;
