type Dataset = {
  category: string
  problems: string[]
}

export const CVRPTW_DATASETS: { category: string; datasets: Dataset[] }[] = [
  {
    category: 'Solomon',
    datasets: [
      {
        category: 'c1',
        problems: [
          'c101',
          'c102',
          'c103',
          'c104',
          'c105',
          'c106',
          'c107',
          'c108',
          'c109',
        ],
      },
      {
        category: 'c2',
        problems: [
          'c201',
          'c202',
          'c203',
          'c204',
          'c205',
          'c206',
          'c207',
          'c208',
        ],
      },
    ],
  },
]
