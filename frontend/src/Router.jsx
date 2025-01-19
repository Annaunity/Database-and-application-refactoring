import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom';
import { HomePage } from './pages/Home.page';
import { MyDrawingsPage } from './pages/MyDrawings.page';
import { MyProfilePage } from './pages/MyProfile.page';
import { DrawingPage } from './pages/Drawing.page';

const router = createBrowserRouter([
  {
    path: '/',
    element: <HomePage />,
  },
  {
    path: '/drawings',
    element: <MyDrawingsPage />,
  },
  {
    path: '/drawings/:drawingId',
    element: <DrawingPage/>,
  },
  {
    path: '/profile',
    element: <MyProfilePage />,
  },
  {
    path: '/*',
    element: <Navigate to="/" replace />
  },
]);

export function Router() {
  return <RouterProvider router={router} />;
}
