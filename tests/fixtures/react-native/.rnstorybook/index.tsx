import { view } from './storybook.requires';
import AsyncStorage from '@react-native-async-storage/async-storage';
export default view.getStorybookUI({ storage: { getItem: AsyncStorage.getItem, setItem: AsyncStorage.setItem } });
