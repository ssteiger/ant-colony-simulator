import {
  BarChart3Icon,
  LayoutDashboardIcon,
  PlayIcon,
  SettingsIcon,
} from 'lucide-react'
import { Link } from '@tanstack/react-router'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/lib/components/ui/sidebar'
import { NavMain } from '~/lib/components/ui/nav-main'
import { NavSecondary } from '~/lib/components/ui/nav-secondary'
import { NavUser } from '~/lib/components/ui/nav-user'

const navMain = [
  { title: 'Dashboard', url: '/dashboard', icon: LayoutDashboardIcon },
  { title: 'Simulations', url: '/simulation', icon: PlayIcon },
  { title: 'Analytics', url: '/analytics', icon: BarChart3Icon },
]

const navSecondary = [
  { title: 'Settings', url: '/settings', icon: SettingsIcon },
]

interface AppSidebarProps {
  variant?: 'inset' | 'sidebar' | 'floating'
}

export function AppSidebar({ variant = 'inset' }: AppSidebarProps) {
  return (
    <Sidebar variant={variant}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton asChild className="data-[slot=sidebar-menu-button]:!p-1.5">
              <Link to="/dashboard">
                <span className="text-base font-semibold">Ant Colony Simulator</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={navMain} />
        <NavSecondary items={navSecondary} className="mt-auto" />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={{ name: 'User', email: '', avatar: '' }} />
      </SidebarFooter>
    </Sidebar>
  )
}
