import { createFileRoute } from '@tanstack/react-router'
import { useNavigate } from '@tanstack/react-router'

const Home = () => {
  const navigate = useNavigate()
  // redirect to dashboard
  navigate({ to: '/dashboard' })
  return null
}

export const Route = createFileRoute('/_authenticated/_app/')({
  component: Home,
})
