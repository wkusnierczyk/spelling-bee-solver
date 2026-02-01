/* eslint-env jest */
// Mock NativeModules.SbsSolver
jest.mock('react-native', () => {
  const rn = jest.requireActual('react-native');
  rn.NativeModules.SbsSolver = {
    solve: jest.fn().mockResolvedValue('{"words":["test"]}'),
    version: jest.fn().mockResolvedValue('0.1.0'),
  };
  return rn;
});

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () => ({
  getItem: jest.fn().mockResolvedValue(null),
  setItem: jest.fn().mockResolvedValue(undefined),
  removeItem: jest.fn().mockResolvedValue(undefined),
}));

// Mock Picker
jest.mock('@react-native-picker/picker', () => {
  const React = require('react');
  const Picker = (props) =>
    React.createElement('Picker', props, props.children);
  Picker.Item = (props) => React.createElement('Picker.Item', props);
  return {Picker};
});

// Mock react-native-safe-area-context
jest.mock('react-native-safe-area-context', () => {
  const React = require('react');
  return {
    SafeAreaProvider: ({children}) => React.createElement('SafeAreaProvider', null, children),
    SafeAreaView: ({children, ...props}) => React.createElement('SafeAreaView', props, children),
    useSafeAreaInsets: () => ({top: 0, right: 0, bottom: 0, left: 0}),
  };
});

