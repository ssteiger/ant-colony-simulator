import type { LucideIcon } from 'lucide-react'
import { Link, useLocation } from '@tanstack/react-router'

import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '~/lib/components/ui/sidebar'

export function NavMain({
  items,
}: {
  items: {
    title: string
    url: string
    icon?: LucideIcon
  }[]
}) {
  const location = useLocation()

  return (
    <SidebarGroup>
      <SidebarGroupContent>
        <SidebarMenu>
          {items.map((item) => {
            const isActive = location.pathname.startsWith(item.url)
            return (
              <SidebarMenuItem key={item.title}>
                <Link to={item.url} className="w-full">
                  <SidebarMenuButton
                    tooltip={item.title}
                    className={isActive ? 'bg-accent text-accent-foreground' : ''}
                  >
                    {item.icon && <item.icon />}
                    <span>{item.title}</span>
                  </SidebarMenuButton>
                </Link>
              </SidebarMenuItem>
            )
          })}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}
