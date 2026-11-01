import { createFileRoute } from '@tanstack/react-router';
export const Route = createFileRoute('/about')({ component: () => <p>About route</p> });
