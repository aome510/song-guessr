import { Float, Spinner, Text, VStack } from "@chakra-ui/react";

const LoadingPopup: React.FC<{ loading: boolean }> = ({ loading }) => {
  if (!loading) {
    return <div />;
  }
  return (
    <Float placement="middle-center">
      <VStack>
        <Spinner />
        <Text>Loading...</Text>
      </VStack>
    </Float>
  );
};

export default LoadingPopup;
