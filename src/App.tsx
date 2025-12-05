import { Route, BrowserRouter, Routes } from "react-router-dom";
import Room from "./Room.tsx";
import HomePage from "./HomePage.tsx";
import {
  Center,
  ChakraProvider,
  defaultSystem,
  Heading,
} from "@chakra-ui/react";

const NotFound = () => <Heading size="6xl">404 - Page Not Found</Heading>;

function App() {
  return (
    <>
      <ChakraProvider value={defaultSystem}>
        <Center minH="100vh">
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/room/:id" element={<Room />} />
              <Route path="*" element={<NotFound />} />
            </Routes>
          </BrowserRouter>
        </Center>
      </ChakraProvider>
    </>
  );
}

export default App;
